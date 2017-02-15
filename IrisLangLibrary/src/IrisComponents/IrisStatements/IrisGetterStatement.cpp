#include "IrisComponents/IrisStatements/IrisGetterStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"


bool IrisGetterStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	const string& strMethodName = m_pGetteredVariable->GetIdentifierString();

	IR_BYTE bWithBlock = m_pBlock ? 1 : 0;

	pCompiler->IncreamDefineIndex();
	pMaker->gtr_def(pCompiler->GetIdentifierIndex(strMethodName, pCompiler->GetCurrentFileIndex()), bWithBlock, pCompiler->GetDefineIndex());

	if (m_pBlock) {
		if (!m_pBlock->Generate()) {
			return false;
		}
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	return true;
}

IrisGetterStatement::IrisGetterStatement(IrisIdentifier* pGetteredVariable, IrisBlock* pBlock) : m_pGetteredVariable(pGetteredVariable), m_pBlock(pBlock)
{
}


IrisGetterStatement::~IrisGetterStatement()
{
	if (m_pGetteredVariable) {
		delete m_pGetteredVariable;
	}
	if (m_pBlock) {
		delete m_pBlock;
	}
}

bool IrisGetterStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ClassBlock && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::AccessorStatementIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "accessor Statement can only be used in Class or Module body.");
		return false;
	}

	if (m_pGetteredVariable->GetType() != IrisIdentifierType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pGetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.");
		return false;
	}

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::MethodBlock);
		
		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}

	return true;
}
