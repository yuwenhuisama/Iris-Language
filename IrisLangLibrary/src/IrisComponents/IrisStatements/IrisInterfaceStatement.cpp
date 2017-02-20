#include "IrisComponents/IrisStatements/IrisInterfaceStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisFatalErrorHandler.h"


bool IrisInterfaceStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push_env();
	pMaker->cre_cenv(2);

	pCompiler->IncreamDefineIndex();
	pMaker->def_inf(pCompiler->GetIdentifierIndex(m_pInterfaceName->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), pCompiler->GetDefineIndex());

	// Interfaces
	if (m_pInterfaces) {
		if(!m_pInterfaces->Ergodic(
			[&](IrisExpression* pExpression) -> bool {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->add_inf();
			return true;
		}
		))
			return false;
	}

	// Block
	if (m_pBlock) {
		m_pBlock->Generate();
	}
	else {
		pCompiler->IncreamDefineIndex();
		pMaker->blk_def(pCompiler->GetDefineIndex());
		pMaker->end_def(pCompiler->GetDefineIndex());
		pCompiler->DecreamDefineIndex();
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	pMaker->pop_env();

	return true;
}

IrisInterfaceStatement::IrisInterfaceStatement(IrisIdentifier* pInterfaceName, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock) : m_pInterfaceName(pInterfaceName), m_pInterfaces(pInterfaces), m_pBlock(pBlock)
{
}


IrisInterfaceStatement::~IrisInterfaceStatement()
{
	if (m_pInterfaceName)
		delete m_pInterfaceName;
	if (m_pInterfaces) {
		m_pInterfaces->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pInterfaces->Clear();
		delete m_pInterfaces;
	}
	if (m_pBlock) {
		delete m_pBlock;
	}
}

bool IrisInterfaceStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisStatementValidateVisitor isvvStatementVisitor;
	IrisExpressionValidateVisitor ievvExpressionVisitor;

	if (!pCompiler->UpperStackEmpty() && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "interface of " + m_pInterfaceName->GetIdentifierString() + " must be defined in Main environment or Module body.");
		return false;
	}

	if (m_pInterfaceName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pInterfaceName->GetIdentifierString() + " is not a CONSTANCE.");
		return false;
	}

	if (m_pInterfaces && !m_pInterfaces->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	}))

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::InterfaceBlock);
		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}
		pCompiler->PopUpperType();
	}

	return true;
}
