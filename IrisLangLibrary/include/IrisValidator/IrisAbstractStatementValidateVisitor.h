#ifndef _H_IRISABSTRACTSTATEMENTVALIDATEVISITOR_
#define _H_IRISABSTRACTSTATEMENTVALIDATEVISITOR_

class IrisStatement;
class IrisAbstractStatementValidateVisitor {
public:
	virtual bool Visit(IrisStatement* pStatement) = 0;
};

#endif
