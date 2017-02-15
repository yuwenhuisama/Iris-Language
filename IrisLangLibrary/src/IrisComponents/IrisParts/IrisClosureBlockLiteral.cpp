#include "IrisComponents/IrisParts/IrisClosureBlockLiteral.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisFatalErrorHandler.h"

bool IrisClosureBlockLiteral::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();

	list<IR_DWORD> lsParameters;

	if (m_pParameters) {
		m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool {
			lsParameters.push_back(pCompiler->GetIdentifierIndex(pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
			return true;
		}
		);
	}

	pCompiler->IncreamDefineIndex();
	if (!m_pVariableParameter) {
		pMaker->cblk_def(lsParameters, -1, pCompiler->GetDefineIndex());
	}
	else {
		pMaker->cblk_def(lsParameters, pCompiler->GetIdentifierIndex(m_pVariableParameter->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), pCompiler->GetDefineIndex());
	}

	if(m_pStatements) {
		if(!m_pStatements->Ergodic(
			[&](IrisStatement*& pStatement) -> bool {
			if (!pStatement->Generate()) {
				return false;
			}
			return true;
		}
			))
			return false;
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	return true;
}

IrisClosureBlockLiteral::IrisClosureBlockLiteral(IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameter, IrisList<IrisStatement*>* pStatements) 
	: m_pParameters(pParameters), m_pVariableParameter(pVariableParameter), m_pStatements(pStatements)
{
}


IrisClosureBlockLiteral::~IrisClosureBlockLiteral()
{
	if (m_pParameters) {
		m_pParameters->Ergodic(
			[](IrisIdentifier*& pIdentifier) -> bool { delete pIdentifier; pIdentifier = nullptr; return true; }
		);
	}
	if (m_pStatements) {
		m_pStatements->Ergodic(
			[](IrisStatement*& pStatement) -> bool { delete pStatement; pStatement = nullptr; return true; }
		);
	}
	if (m_pVariableParameter) {
		delete m_pVariableParameter;
	}
}

bool IrisClosureBlockLiteral::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	IrisExpressionValidateVisitor isvvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (m_pParameters && !m_pParameters->Ergodic([&](IrisIdentifier*& pIdentifier) -> bool {
		if (pIdentifier->GetType() != IrisIdentifierType::LocalVariable) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + pIdentifier->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pStatements && !m_pStatements->Ergodic([&](IrisStatement*& pStatement) -> bool {
		if (!pStatement->Accept(&isvvStatementVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pVariableParameter && m_pVariableParameter->GetType() != IrisIdentifierType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pVariableParameter->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
		return false;
	}

	return true;
}
