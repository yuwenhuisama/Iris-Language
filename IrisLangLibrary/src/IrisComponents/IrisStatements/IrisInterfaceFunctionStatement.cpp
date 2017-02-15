#include "IrisComponents/IrisStatements/IrisInterfaceFunctionStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include <list>
using namespace std;

bool IrisInterfaceFunctionStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	unsigned int nNameIndex = pCompiler->GetIdentifierIndex(m_pFunctionName->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

	list<IR_DWORD> lsParameters;
	if (m_pParameters) {
		if(!m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool{
			lsParameters.push_back(pCompiler->GetIdentifierIndex(pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
			return true;
		}
		))
			return false;
	}
	
	IR_DWORD dwVariablePrameterIndex = -1;
	if (m_pVariableParameter) {
		dwVariablePrameterIndex = pCompiler->GetIdentifierIndex(m_pVariableParameter->GetIdentifierString(), pCompiler->GetCurrentFileIndex());
	}

	pMaker->def_infs(nNameIndex, lsParameters, dwVariablePrameterIndex);

	return true;
}

IrisInterfaceFunctionStatement::IrisInterfaceFunctionStatement(IrisIdentifier* pFunctionName, IrisList<IrisIdentifier*>* pParameters, IrisIdentifier* pVariableParameters) : m_pFunctionName(pFunctionName), m_pParameters(pParameters), m_pVariableParameter(pVariableParameters)
{
}


IrisInterfaceFunctionStatement::~IrisInterfaceFunctionStatement()
{
	if (m_pFunctionName)
		delete m_pFunctionName;
	if (m_pParameters) {
		m_pParameters->Ergodic([](IrisIdentifier*& x) -> bool { delete x; x = nullptr; return true;  });
		m_pParameters->Clear();
		delete m_pParameters;
	}
	if (m_pVariableParameter)
		delete m_pVariableParameter;
}

bool IrisInterfaceFunctionStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	if (pCompiler->GetTopUpperType() != IrisCompiler::UpperType::InterfaceBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "interface function of" + m_pFunctionName->GetIdentifierString() + "must be defined in Interface body.");
		return false;
	}
	
	if (m_pParameters) {
		if (!m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool {
			if (pIdentifier->GetType() != IrisIdentifierType::LocalVariable) {
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + pIdentifier->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
				return false;
			}
			return true;
		}
		))
			return false;
	}

	if (m_pFunctionName->GetType() != IrisIdentifierType::LocalVariable
		|| m_pFunctionName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pFunctionName->GetIdentifierString() + " is NOT a valid method name.");
		return false;
	}

	if (m_pVariableParameter && m_pVariableParameter->GetType() != IrisIdentifierType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pVariableParameter->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
		return false;
	}

	return true;
}
